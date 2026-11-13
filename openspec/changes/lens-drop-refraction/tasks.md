# Tasks

- [x] 1. Before captures at the owner's beach (`--at 12.948 8.135 --rain 1`, four directions at noon and one at midnight), taken from the unchanged shader
- [x] 2. `water.wgsl`: the drop field returns its coverage, lens height and trail, with the layer's drop and beads apart; the lens offset from the lens height's slope, toward the centre, the aspect applied, capped at `bend·R_drop`. Used by the rain drops and the emerge drips
- [x] 3. `rain_lens_bend` replaces `rain_lens_refract` (config, validation 0..=2, `.ron`, the `lens.y` lane)
- [x] 4. The edge and the light under it: `rain_lens_rim` and `rain_lens_glint` (config, validation, `.ron`, a new `lens_look` lane), the sky's mean radiance on the lower edge
- [x] 5. Tests: a GPU run of the shipped drop section (within the cap, toward the centre, inside a drop, lit below, alike across and down), the section compiling on its own, and the config's ranges
- [x] 6. After captures with the same arguments, side-by-sides, 1:1 crops, and the README with each shot's place, time, sun and weather
- [ ] 7. The owner judges the drops on the beach shots, and in a real-time walk in rain (a `--real-time` capture or by hand), before the requirements move into `openspec/specs/world/weather`
