# Tasks

The owner, 2026-09-30: "we should probably implement some sort of CSM
shadows for the planet too, so we can read sun direction a little better".

## 1. Measure first

- [ ] 1.1 Before captures at a low sun: 08:00 and 17:30 over a ridge, down Holbrook's lane, among trees, and from 1 km up, with the towns mockup's lane at the same hour beside them. Verify: the shots are in `docs/screenshots/sun-shadows/` with the flags that took them.
- [ ] 1.2 A `planet_shadow` GPU span in `RenderDiagnosticsPlugin`'s set, so `--frame-log` and `tools/perf_suite.py` report the pass. Verify: an app test that the span's name is registered.

## 2. The cascades (`pbd_core::shadow`)

- [ ] 2.1 Cascade splits by view (on foot, and by altitude up to the horizon's distance); bounding spheres with radii rounded to 1 m; centres snapped to a texel in the light's frame; near planes pulled to the R + 320 m shell toward the sun; matrices in f64. Verify: core tests that each cascade covers its frustum slice, that a move under a texel changes nothing and a turn in place changes nothing, and that a caster on the shell between the slice and the sun is inside the near plane.
- [ ] 2.2 The refresh schedule: every frame, every other and every fourth, with a time jump redrawing all four; a cascade keeps the matrices it was drawn with. Verify: core tests of the schedule, and that a jump redraws all.

## 3. The shadow pass

- [ ] 3.1 A four-layer 2048² depth array, and a render-graph node that draws the cascades before the planet's main draw. Verify: an app test that the node and its texture exist in the render graph.
- [ ] 3.2 A light-view cull in `planet_visibility.wgsl`: the cascade's box, no horizon cull, the camera's detail levels. Verify: a GPU test (lavapipe) that a record inside a cascade's box and behind the camera is kept, and one outside it is not.
- [ ] 3.3 The depth-only planet pipeline for terrain, columns and trees, with a slope-scaled bias. Verify: naga validation of the shader, and the pipeline built in the app test.

## 4. Receivers

- [ ] 4.1 `sun_shadow` in a shared WGSL module: the cascade by view depth, a 10% blend, a 3 × 3 comparison kernel, and a normal offset. Verify: naga validation, and a GPU test where a wall's shadow reads darker than the ground in front of it.
- [ ] 4.2 The terrain takes it on its direct term only. Verify: the shader-constant tests pin the new line, and a GPU test that a shadowed face open to the sky matches a sunless one's sky fill.
- [ ] 4.3 Field-lit objects take it (`sun_up = sunlight * sky * sun_shadow`). Verify: `field_light` tests extended to a shadowed point.

## 4b. Indoors, and faces in shade (design decision 7)

- [ ] 4b.1 Bevy's ambient set to the terrain's daytime sky fill, following the clock. Field-lit objects add it rather than blend it away. Verify: the shader-constant test holds the one fill, a `field_light` test that a face turned from the sun reads the sky fill at noon, and before and after shots of a house front facing away from the sun.
- [ ] 4b.2 The cutter tags each building's inside faces, and the town draws them as their own meshes with a sky of 0.3 when a door stands open and 0.2 when shut, taking no direct sun. Verify: a core test that every inner wall face, floor, ceiling and stair is tagged inside and no outer face is; the slice 2b views retaken, beside the before and the mockup's.

## 5. Towns cast

- [ ] 5.1 A town's meshes as one static caster buffer, drawn by a position-only depth pipeline. Verify: an app test that a built town registers its caster, and a capture of a house's shadow across the lane.

## 6. The sun ends at the horizon

- [ ] 6.1 `preview-scale-and-shader-parity` task 7: `sunlight` for direct light, specular and glints, `twilight` for ambient, fog and rim. Verify: the shader-constant tests pin both curves; that change's task 7 is ticked, naming this change.

## 7. The owner's check

- [ ] 7.1 After captures of 1.1's views, beside the before, and the mockup's lane at the same hour. Verify: `docs/screenshots/sun-shadows/README.md` names each shot.
- [ ] 7.2 `tools/perf_suite.py` on the owner's hardware: `walk`, `far-side` and `clouds`, before and after, interleaved; the report in `docs/benchmarks/`. A cloud session says the cost was not measured. Verify: the report, and the design's decision 6 holds or its fallbacks are taken.
- [ ] 7.3 The owner records a walk and a fly-in in real time (`obs-record`) to judge crawl and shimmer, which fixed-step captures cannot show. Verify: the owner's word, quoted in `proposal.md`. Sync `planet/light`, and archive.
