# Proposal

## Why

The vehicle renderer still assembles untextured cuboids and thin hull surfaces.
It misses the prototypes' shaped fuselage, cockpit, nacelles, open canoe and
rig details. Replace these with editable Blender models while preserving the
dimensions and moving pivots that the physics and handling tests establish.

## What Changes

- Author low-poly Kestrel, Tern and Loon meshes, UVs and pixel-art PNG atlases
  in Blender 4.4. Commit `.blend` sources, exported `.glb` files, region
  manifests and reproducible authoring inputs/scripts under `assets/models/`.
- Use 16 pixels per metre throughout, padded UV charts and nearest filtering.
  Retain white/coral Kestrel, coral-deck/pale-sail Tern and teal/pale-paddle Loon.
- Preserve the moving hierarchy and add modeled Kestrel control surfaces driven
  by the actual physics deflections. Keep every model render-only.
- Convert the supported static glTF subset into Bevy meshes/materials at load
  time, keeping this workspace's Bevy glTF feature disabled.
- **Compatibility change:** geometry configuration changes require regeneration
  and export from Blender. Validate actual exported meshes, named parts, pivots,
  axes and dimensions against config; fail on divergence. No runtime reshaping.
- Inspect Blender viewport screenshots. All game captures and game appearance
  review belong to Claude, as explicitly requested by the owner.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player/vehicles`: authored, textured render models with tested geometry and
  animation contracts, replacing procedural visual construction and documenting
  the regeneration requirement.

## Impact

`pbd-app` gains a narrow GLB conversion/cache module and exported-art validation;
`draw.rs` retains state-driven placement and loses the primitive builders as
each craft lands. The cached `gltf` parser is an app-only dependency. Core gains
only engine-independent control-deflection telemetry and an authoring-input
example; no Bevy or Blender dependency enters its runtime. Physics, controls,
save identity and immediate mouse look retain their existing authority.

The owner explicitly requested planning followed by implementation, in that
order, without another question. Commit this write-up alone after the handling
commits, then implement in one focused commit per craft. Keep unperformed game
review visible in the handling handoff and this change's evidence.
