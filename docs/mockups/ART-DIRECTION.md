# Mockup art direction

The owner, 2026-11-12, on the fourteen mockups of 2026-11-11: "all the art needs
rework in the mockups you did today". Before that: "why redraw tiles we already
have texture for?", and "the card game rubens solitaire collection uses a
different art style, can you adopt that art style instead?".

A mockup is a picture of **this game**, not a generic three.js scene. Put a
mockup screenshot beside an in-game one (`docs/screenshots/`, e.g.
`town-gates-after-ashenstead-north-outside.png`,
`cities-in-the-world/game-4a-dunmouth-above-night.jpg`,
`bigger-biomes/biome-fields.jpg`) and it should look like the same game.

## The look of the game

- **Everything is blocks on the hex grid.** Ground is hex columns: 2.833 m
  across, 1 m layers, flat tops and vertical walls. Buildings are straight-walled
  boxes with pitched roofs; trees are the game's hex-prism trees, or the foliage
  playground's branch-and-card trees. No smooth spheres, lathed blobs or capsule
  bodies standing in for things.
- **Pixel textures, sampled nearest, on every surface.** A 32x32 texture repeats
  at its `repeat_m`. No surface is a flat vertex colour unless the game draws it
  that way (light, sky, water tint).
- **Light like the game.** Sun from above, soft ambient. At night each lamp
  throws a warm pool on the ground and walls round it, windows glow, and the moon
  is dim (CLAUDE.md, "Mockups are lit at night"). **Night is dark:** ground
  outside a pool reads near-black, pools are tight and warm with dark gaps
  between them, as in `cities-in-the-world/game-4a-dunmouth-above-night.jpg`.
  A night scene where the whole ground is lit is wrong.
- **Its HUD.** Where a mockup shows UI, it uses the game's HUD look: the hotbar
  slots, the compass bar, the monospace debug line, pixel icons at whole-number
  scales.

## Where the art comes from, in this order

1. **The game's own asset**, copied byte-identical into
   `docs/mockups/<page>/game/` with a README naming each source:
   - terrain: `assets/tilesets/atlas.png` (4x4 biome sheets of 4x4 tiles; sheet
     order in `crates/pbd-app/src/planet_terrain.rs`, per-biome slot in
     `planet.rs`, top/side/earth/stone tiles as `planet_surface.wgsl` picks them);
   - materials: `assets/textures/settlement/*.png` (`manifest.ron` for
     `repeat_m`);
   - item icons: `assets/items/{fish,lights,tools}/*.png`;
   - vehicles: `assets/models/vehicles/<craft>/<craft>.glb` with `atlas.png`
     (load with GLTFLoader);
   - towns: `assets/settlements/v1/*.json` templates, built the way
     `crates/pbd-app/src/towns.rs` builds them, where a mockup shows a town.
2. **New art in the solitaire collection's style**, drawn with
   `.claude/skills/balatro-pixel-art` (Sweaft-64 palette, small native canvases,
   1 px outline on sprites, hard alpha), only for what the game does not have.
   New 3D things (an animal, a station module, a ship part, a fish) are built
   from boxes and prisms the way the game builds things, and textured with
   pixel textures drawn by that kit, never left flat-coloured.
3. Nothing else. The crayon kit (`hand-drawn-illustration`) is retired here.

Every page says which art is the game's and which is mockup-only.

## Rendering rules

- three.js textures: `NearestFilter`, no mipmaps; cut-outs `alphaTest` 0.5,
  depth-written, never blended; no two cards coplanar.
- 2D sprites: `image-rendering: pixelated`, whole-number scales only.
- Cache-bust texture URLs (`?v=<n>`) when they change.
- Nothing spins on its own: no idle camera orbit, no turntable, no parked
  rotor turning (CLAUDE.md, "Nothing in a mockup spins on its own").

## The gate before a page is published

1. Screenshot the page on the GPU (Edge with `--use-angle=d3d11 --enable-gpu
   --ignore-gpu-blocklist`, served over `python -m http.server`; never
   SwiftShader on a machine with a GPU, CLAUDE.md "Render on the GPU") by day
   and by night.
2. Put it beside one of the in-game screenshots above and read both. If the
   mockup would look out of place in the game, it is not done.
3. Contact-sheet new sprites beside `.claude/skills/balatro-pixel-art/references/`.
4. Then republish the same artifact.
