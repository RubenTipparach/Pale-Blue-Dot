# Tasks

## 1. Planning and handling handoff

- [x] 1.1 Commit the passing handling work and record remaining game/subjective review in its handoff; verify the branch and focused diffs.
- [x] 1.2 Read prototypes, configs, implementation and specs; write proposal/design/deltas/tasks including the density rule, pivot table, loading cost and regeneration decision.
- [x] 1.3 Validate OpenSpec and run fmt, both offline release package suites and all-target Clippy; commit only the vehicle-models planning artifacts.

## 2. Common pipeline and Kestrel commit

- [ ] 2.0a Generate material swatches with the image-generation tool, normalize to 32-square / 2-metre repeats with nearest sampling and shared ramps, check wraps, commit source art/provenance, and compose them into each atlas with unique pixel overlays.
- [x] 2.0 Apply owner/Claude texture review while preserving approved geometry: paint 3-5-shade material ramps and craft-specific pixel details, tightly pack with declared tile reuse, and verify actual color counts, nearest-upscaled atlases and neutral Blender renders for each craft.

- [x] 2.1 Export authoritative config, foil frames and hull loft authoring inputs through a core example; document and run the regeneration command with no engine dependency in core.
- [x] 2.2 Build the Kestrel meshes, separate moving objects, exact-density UVs and pixel atlas in Blender; save `.blend`, `.glb`, PNG and region manifest; inspect Blender viewport screenshots.
- [x] 2.3 Convert and cache the supported GLB subset in the app, preserving names/hierarchy/transforms and explicit nearest sampling; test loading the actual Kestrel artifact into ECS.
- [x] 2.4 Expose actual core surface-deflection telemetry and drive the modeled surfaces plus existing nacelles/rotors/crew; test authoritative deflections and loaded moving pivots.
- [ ] 2.5 Replace the procedural Kestrel area/axis test with exported-art validation, including pivot/dimension/UV/texture checks and mutated-config rejection; record measured counts/sizes and migrate only the shared/Kestrel requirements with passing tests.
- [ ] 2.6 Run fmt, both offline release package suites, all-target Clippy and OpenSpec validation; review the diff and commit the Kestrel and common pipeline.

## 3. Tern commit

- [ ] 3.1 Model the configured hull/deck/cockpit, rig, exact-area sail and appendages in Blender with separate boom/sail/rudder/tiller objects; save sources/export/PNG/manifest and inspect viewport screenshots.
- [ ] 3.2 Replace procedural Tern construction and test actual exported hull/foil/rig dimensions, axes, crew eye, animation hierarchy, UV density and stale-config failures.
- [ ] 3.3 Record artifact measurements and migrate the Tern requirement with its implementation/tests; run fmt, both offline release package suites, all-target Clippy and OpenSpec validation, then commit the Tern.

## 4. Loon commit

- [ ] 4.1 Model the open configured hull, lining/gunwales/seats/thwarts, skeg and shaped pale paddle in Blender; save sources/export/PNG/manifest and inspect viewport screenshots.
- [ ] 4.2 Replace procedural Loon construction and test actual hull/skeg/paddle dimensions, crew eye, stroke/recovery/zero-speed rudder placement, UV density and stale-config failures; remove the final primitive vehicle builders.
- [ ] 4.3 Record artifact measurements and migrate the Loon requirement with its implementation/tests; run fmt, both offline release package suites, all-target Clippy and OpenSpec validation, then commit the Loon.

## 5. Integration evidence and handoff

- [ ] 5.1 Build the final release app offline and check its help without taking game captures; confirm all three registered assets load and all handling/frame/input regressions remain green.
- [ ] 5.2 Record produced files, density, loading behavior, Blender review evidence and unverified game appearance; verify clean current branch and no push.
- [ ] 5.3 Claude's reserved review: in-game craft/seat/translated-frame captures, lighting, clipping, distance readability and owner art assessment. Keep this task open until evidence is supplied.
