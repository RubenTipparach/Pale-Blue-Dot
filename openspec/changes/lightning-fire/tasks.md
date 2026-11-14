## 1. Where it lands

- [ ] 1.1 The landing search:
  - the tallest solid top in 24 m from the column tier, the height field
    where the tier is not loaded;
  - no water, no covered cells.

  Verify: unit tests for the crown, the water and determinism.
- [ ] 1.2 The `STRIKE` log line and `--strike-at LAT LON [SECONDS]`. Measure
  how often a natural strike lands within 3 km of a player standing in a
  forced storm. Write the number into the design before deciding whether to
  raise the rate.

## 2. The burst and the fire

- [ ] 2.1 The bolt ends at the landing point, with an impact burst of sparks
  and a steam puff.
- [ ] 2.2 `Fires`:
  - life from the rain, the material rule and `MAX_FIRES`;
  - flames, smoke and steam quads in the weather mesh;
  - `FIRE_OUT`.

  Verify: unit tests for life and material.
- [ ] 2.3 The storm lights:
  - the flash term becomes `STORM_LIGHTS` point lights (the cloud, the
    impact, the fires) in the ground, rain and fog passes;
  - skipped at intensity 0.

  Verify: a capture at night by a burning tree.

## 3. Close

- [ ] 3.1 Captures in `docs/screenshots/lightning-fire/`, the storm perf
  scenario before and after, and a real-time video.
- [ ] 3.2 The spec moved into `openspec/specs/world/weather/`, and
  `openspec validate --all`.
