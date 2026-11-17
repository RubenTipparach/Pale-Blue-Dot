# Underwater shafts: light in the water, not in the scene

## Why

The owner, 2026-11-17, from under shallow water near a coast: "underwater
rays have this weird depth thing happening to them, dont seem like they are
rendering right, as it appears to be modulating fog depth color". In the
screenshot the bright diagonal shaft carries the scene's geometry in it:
where it crosses the view, its brightness changes with the depth of the
blocks behind, so block silhouettes and hard depth steps (dark wedges and
edges round the pillars) show inside the shaft, where a shaft should be a
soft volume of light in the water between the eye and the scene.

The cause is in `shafts()` in `assets/shaders/water.wgsl` (design 1): the
shaft march is spread over the distance to the scene, so every pixel whose
scene is nearer than the shafts' reach (40 m) samples the shaft pattern at
different points from its neighbour. Across a depth step the two pixels see
unrelated parts of the pattern, and the scene's outline is drawn into the
light.

## What changes

- The shafts' samples stand at fixed distances along the view ray, set by
  the shafts' reach alone, and the scene's distance only clips the march:
  each sample weighs the part of its own stretch that lies in front of the
  scene, and the one stretch the scene cuts is sampled in front of it. The
  light gathered is then the same in-scatter for every pixel up to where its
  scene stands, and changes smoothly with that distance.
- The march takes sixteen samples instead of ten. Laid over the full
  40 m, ten left the columns near the surface blurred to their mean (design
  2).
- Nothing else in the underwater view changes: the strength, reach,
  pattern, its sway, the phase toward the sun, the murk, the fog, the snow,
  Snell's window and the surface from below are as they were. The fall look
  (`water.wgsl` falling faces) is not touched.

## Impact

- `assets/shaders/water.wgsl`: `shafts()` only.
- `crates/pbd-app/src/shader_tests.rs`: a test that the march's sample
  distances do not come from the scene's distance.
- `openspec/specs/world/water-environments`: one added requirement.
- Frame cost: up to six more shaft samples on an underwater pixel that sees
  40 m of open water; a pixel whose scene is nearer stops at it. Not
  measured (a cloud session, lavapipe); the owner's desktop run is task 5.
