# Proposal: the sun casts shadows

## Why

The owner (2026-09-30): "we should probably implement some sort of CSM
shadows for the planet too, so we can read sun direction a little better".

Nothing in the game casts or receives a sun shadow today. A slope facing
the sun is brighter than one facing away, and that is all the picture says
about where the sun is. A ridge does not shade the valley behind it, a
house does not shade the lane, and a tree does not shade the grass.

**What was measured** (a read-only survey of the code, 2026-09-30):
- **Bevy's sun has shadows off.** The one `DirectionalLight` has
  `shadows_enabled: false`, and no `CascadeShadowConfig` is set
  (`desktop/scene.rs:20-29`).
- **The terrain can't take part in Bevy's shadows.** It is a custom
  vertex-pulled pipeline in the `Transparent3d` phase, with its own group 0
  (`planet.rs:917-960`, 849-868).
  - It has no Bevy view bindings, so it cannot read Bevy's shadow maps.
  - It is not a `Mesh3d`, so it casts nothing into them.
- **What passes for sun occlusion is sky openness, and it ignores where
  the sun is:**
  - the terrain's per-cell sky openness;
  - the voxel sky field in the column tier;
  - the field-lit objects' `sky` term (`field_lit.wgsl:66-94`,
    `lamps-and-lanterns` decision 14).

  So a surface open to the sky is fully sunlit even with a ridge between it
  and the sun.
- **Cloud shadow is already done**, by `cloud_sun`. It reads the weather
  map along the sun ray (`planet_surface.wgsl:47-62`), and this change keeps
  it.
- **The towns mockup casts real sun shadows**: three.js PCF soft shadows on
  a 2048² map (`docs/mockups/towns.html:3552-3560`). The game should match
  (`cities-in-the-world`, "Everything in the approved towns mockup is in the
  game").
- **A held piece of work is waiting for this.**
  `preview-scale-and-shader-parity` task 7 splits direct sun from twilight
  fill, and waits "for the shadows that make it matter". A shadow cast by a
  sun seven degrees below the horizon is plainly false.

## What changes

- **Sun shadow cascades for the whole scene**, the planet's own:
  - four cascades fitted to the camera's view, deeper when flying, drawn
    from the one published sun;
  - stable as the camera moves: snapped to their texels, sized by bounding
    spheres;
  - refreshed less often far out, since the sun moves 0.125 degrees a
    second.
- **What casts:** the terrain, the column tier, trees, and the towns. The
  planet draws depth into the cascades from its own records. A town adds
  its meshes as a static caster.
- **What receives:** the terrain, trees, clutter and columns (in
  `planet_surface.wgsl`), and every field-lit object: towns, vehicles, drops
  and fish (in `field_lit.wgsl`).
  - The shadow scales the direct sun only.
  - Sky fill, lamps and the ambient floor stay, so a shadow is never black.
- **The sun's direct term ends at the horizon.** `preview-scale-and-shader-
  parity` task 7's split lands with the shadows. Direct light and glints
  take the tight `sunlight` curve, and fill takes `twilight`.
- **Not in this change:**
  - clouds casting into the cascades, since the cloud map already shades
    the ground;
  - water receiving shadows;
  - vehicles and fish casting (they receive).

## Capabilities

- `planet/light`: "The sun casts shadows" is added. "Nothing unlit is fully
  black" gains a scenario, a face in shadow.

## Impact

- **Code:**
  - `pbd-core`: new `shadow` module for the cascade fitting, pure and
    tested.
  - `pbd-app`: `planet.rs` (a shadow node and pipeline),
    `planet_visibility.wgsl` (a light-view cull), `planet_surface.wgsl`,
    `field_light.rs` and `field_lit.wgsl` (receivers), `towns.rs` (static
    casters).
- **Frame cost.** It is the risk, and it is measured on the owner's hardware
  with `tools/perf_suite.py`, old and new interleaved. A cloud session has no
  GPU, so the cost cannot be measured there.
- **Saves.** None: shadows are drawn, derived state.
